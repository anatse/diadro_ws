#docker save diadro > diadro.tar

#push to dockeer-hub 
docker tag "$(docker images -a -q diadro:latest)" anatolse/myrepo:diadro
docker push anatolse/myrepo:diadro