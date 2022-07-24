# ticketland-api

Start Minio
===


```
docker run -d  \
  -p 9000:9000 \
  -p 9001:9001 \
  -e "MINIO_ROOT_USER=OSJ90KMK8FNEILHQOKMS" \
  -e "MINIO_ROOT_PASSWORD=lM02Cnff9RlfQ9cK+tRg5oP3R27glCnPESJ7siW+" \
  quay.io/minio/minio server /data --console-address ":9001"
```
