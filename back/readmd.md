topic
개인 메시지 전달 토픽
ws_id:{ws_id}

룸 이벤트 메시지 전달 토픽
room_id:{room_id}

룸 버전 개인 메시지 전달 토픽
room_id:{room_id}:ws_id:{ws_id}

룸 목록 메시지 전달 토픽
room_list_update






room:<room_id>:info
<json>
room info
{
    host: <ws_id>
    users: [{},{},{}],
    
}

room:<room_id>:chats
<redis list,  json>
room chats{
    
}

room:<room_id>:events
<redis list,  json>
room even
{

}

방생성
방제거(방제거는 마지막 사람이 나가면)

유저 입장
유저 퇴장

호스트 변경

방 전부 나갈시 파괴

채팅



기능
방생성
    방 데이터 개인 퍼블리시
    방 정보 구독
    방목록 업데이트 퍼블리시

방입장
    방 데이터 개인 퍼블리시
    방 정보 구독
    방 정보 업데이트 퍼블리시
    인원수 방목록 업데이트 퍼블리시

방퇴장
    방 정보 구독 해제
    인원수 방목록 업데이트 퍼블리시

방목록 구독
    방 목록 개인 퍼블리시
    방 목록 구독

방목록 구독 해제
    방 목록 구독 해제

방채팅
    방 에 채팅 내용 퍼블리시


ws 연결
    lobby subscribe
    만약 로그인되었다면
        ws user login 시도
            성공시 lobby enter 해줌
            모든 기능 가능(로비채팅, 방 진입, 방 만들기)

            로그아웃 하면
                로비나가기,
                user unauthenticated 변경


    만약 로그인 안되었다면
        그대로유지
        로비 채팅이나 방만들기 이런거 못함

ws 끈어지면
    해당 ws_id 와관련된 room, lobby 다 나가기 처리


ws connected
ws disconnected

user login =>
user loginout =>

lobby enter =>
lobby leave =>

room enter =>
room leave =>