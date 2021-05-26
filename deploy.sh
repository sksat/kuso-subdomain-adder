#!/bin/bash
cd $(dirname $0)

function update(){
	local is_running=`docker-compose ps --services --filter "status=running"`

	echo "pulling repository..."
	git pull
	echo "pulling new images..."
	docker-compose pull
	if [[ "$is_running" != "" ]]; then
		echo "services are still running!"
		docker-compose down
	fi
}

function up(){
	docker-compose up $1
}

if [ $# -ne 1 ]; then
	update
	up -d
	exit 0
fi

case $1 in
	"install") cp ./kuso-subdomain-adder.service /etc/systemd/system/ ;;
	"uninstall") systemctl stop kuso-subdomain.service; rm /etc/systemd/system/kuso-subdomain-adder.service ;;
	"start") update; up > /dev/null ;;
	"stop") docker-compose down ;;
	"update") update ;;
esac
