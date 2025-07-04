use std::collections::HashMap;

use crate::ws_world::model::{TopicId, WsId};

#[derive(Debug)]
pub struct WsPubSub {
    pubsub: HashMap<TopicId, tokio::sync::broadcast::Sender<String>>,
    // topic broadcast 채널을 각 유저 ws sender와 연결시켜놓은 태스크 핸들
    user_topic_handle: HashMap<WsId, WsWorldUserTopicHandle>,
}
impl WsPubSub {
    pub fn new() -> Self {
        Self {
            pubsub: HashMap::new(),
            user_topic_handle: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct WsWorldUserTopicHandle {
    ws_send_tx: tokio::sync::mpsc::UnboundedSender<String>,
    topics: HashMap<TopicId, tokio::task::JoinHandle<()>>,
}

impl WsPubSub {
    pub fn get_pubsub_info(&self) -> Vec<(TopicId, usize)> {
        self.pubsub
            .iter()
            .map(|(topic, sender)| (topic.clone(), sender.receiver_count()))
            .collect::<Vec<_>>()
    }

    pub fn get_user_topics(&self) -> Vec<(WsId, Vec<TopicId>)> {
        self.user_topic_handle
            .iter()
            .map(|(ws_id, h)| (ws_id.clone(), h.topics.keys().cloned().collect::<Vec<_>>()))
            .collect::<Vec<_>>()
    }
}

impl WsPubSub {
    pub fn create_topic_handle(
        &mut self,
        ws_id: WsId,
        ws_send_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) {
        self.user_topic_handle.insert(
            ws_id,
            WsWorldUserTopicHandle {
                ws_send_tx: ws_send_tx,
                topics: HashMap::new(),
            },
        );
    }

    pub fn delete_topic_handle(&mut self, ws_id: &WsId) {
        if let Some(mut user_topic_handle) = self.user_topic_handle.remove(ws_id) {
            for (_, handle) in user_topic_handle.topics.drain() {
                handle.abort();
            }
        }
    }
}

impl WsPubSub {
    /// 토픽 에대한 구독을 진행한다.
    /// 1. 해당 토픽에대한 broadcast sender 가 존재하면 그거 구독하고
    ///        존재하지 않으면 broadcast sender 생성후 구독한다.
    /// 2. 구독하고 나온 broadcast receiver를 recv 하는 태스크를 만들어서
    ///       해당 broadcast receiver로 나온 메시지들을 사용자 ws_sender에 그대로 전달한다.
    pub fn subscribe(&mut self, ws_id: &WsId, topic: &TopicId) {
        let broad_receiver = if let Some(broad_sender) = self.pubsub.get(topic) {
            let broad_receiver = broad_sender.subscribe();
            broad_receiver
        } else {
            let (s, _) = tokio::sync::broadcast::channel::<String>(128);
            let broad_receiver = s.subscribe();
            self.pubsub.insert(topic.clone(), s);
            broad_receiver
        };

        match self.user_topic_handle.get_mut(ws_id) {
            Some(user_topic_handle) => match user_topic_handle.topics.get(topic) {
                Some(_) => {
                    tracing::info!(
                        "pubsub_subscribe, ws_id: {ws_id:?}, topic: {topic:?} is exists, subscrie ignored"
                    );
                }
                None => {
                    user_topic_handle.topics.insert(
                        topic.clone(),
                        Self::subscribe_recv_task(
                            broad_receiver,
                            user_topic_handle.ws_send_tx.clone(),
                        ),
                    );
                }
            },
            None => {
                tracing::info!(
                    "pubsub_subscribe, user_topic_handle ws_id: {ws_id:?} is empty, ignored"
                );
            }
        }
    }

    /// topic_subscribe 할때 생성하는 태스크
    /// broad_receiver -> ws_sender 로 단순 전달
    /// 해당 핸들을 직접 abort 하거나, RecvErr::Closed가 넘어오지 않는이상
    /// 계속 await 하는 태스크로 남아 있게된다.
    fn subscribe_recv_task(
        mut broad_receiver: tokio::sync::broadcast::Receiver<String>,
        ws_sender_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match broad_receiver.recv().await {
                    Ok(msg) => {
                        if !ws_sender_tx.is_closed() {
                            if let Err(err) = ws_sender_tx.send(msg) {
                                tracing::warn!("topic_subscribe_recv_task send err:{err:?}")
                            }
                        }
                    }
                    Err(err) => match err {
                        tokio::sync::broadcast::error::RecvError::Closed => {
                            tracing::warn!("err: closed: {err:?}");
                            break;
                        }
                        tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                            tracing::warn!("err: lagged: {err:?}");
                        }
                    },
                }
            }
        })
    }

    /// pubsub을 구독하고 있는 토픽 핸들을 제거한다.
    /// 반드시 이때 핸들을 abort 시켜줘야한다.
    pub fn unsubscribe(&mut self, ws_id: &WsId, topic: &TopicId) {
        match self.user_topic_handle.get_mut(ws_id) {
            Some(user_topic_handle) => match user_topic_handle.topics.remove(topic) {
                Some(handle) => {
                    handle.abort();

                    // TODO: 고민, 바로 cleanup vs 게으른 cleanup
                    // let cloned_world_sender = self.world_sender.clone();
                    // tokio::spawn(async move {
                    //     let _ = handle.await;
                    //     let _ = cloned_world_sender.send(WsWorldCommand::Pubsub(Pubsub::Cleanup));
                    // });
                }
                None => {
                    tracing::info!(
                        "pubsub_unsubscribe, ws_id: {ws_id:?}, topic: {topic:?} is empty, unsubscribe ignored"
                    );
                }
            },
            None => {
                tracing::info!(
                    "pubsub_unsubscribe, user_topic_handle ws_id: {ws_id:?} is empty, ignored"
                );
            }
        }
    }

    /// 해당 pubsub 토픽에 메시지를 퍼블리시 한다.
    pub fn publish(&self, topic: &TopicId, msg: impl Into<String>) {
        if let Some(broad_sender) = self.pubsub.get(topic) {
            if broad_sender.receiver_count() > 0 {
                if let Err(err) = broad_sender.send(msg.into()) {
                    tracing::warn!("topic_publish {err:?}, topic: {topic:?}");
                }
            }
        } else {
            // pubsub 에 없으면 publish 안한다.
            // tracing::warn!("topic_publish missing topoic:{topic}, msg: {msg:?}");
        }
    }

    /// topics 리스트 받아서 퍼블리시
    #[allow(dead_code)]
    pub fn publish_vec(&self, topics: &[TopicId], msg: &str) {
        for topic in topics {
            self.publish(topic, msg);
        }
    }

    /// pubsub broadcast receiver 가 없는 sender들을 제거한다.
    pub fn pubsub_cleanup(&mut self) {
        self.pubsub
            .retain(|_topic, sender| sender.receiver_count() > 0);
    }
}
