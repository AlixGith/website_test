#!/bin/bash

set -eu

# sudo docker build . -f cargo_build.dockerfile -t cargo_build
#sudo docker ps -a | grep -q cargo_build
#echo $?
if sudo docker ps -a | grep -q cargo_build ; then
	echo ok
	sudo docker start -ai cargo_build
else
	sudo docker run -it --name cargo_build -v "$(pwd):/app" -v ~/.cargo:/home/user/.cargo cargo_build
fi

branch="$(git branch --show-current)"

cp target/release/website2 ~/"git/jean-cloud/services/provisioning/roles/deploy_all/files/bin/omarustwebsite-$branch"
