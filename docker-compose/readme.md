compose up 전

1. docker network 생성
```
docker network create t3tris-shared-net
```

2. infra up

3. app up

4. nginx/nginx.prod.conf 를 ~/t3tris-infra-docker-volume/nginx/conf.d/ 폴더 아래에 복사

5. docker exec nginx nginx -s reload 해주기

