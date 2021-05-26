#!/bin/bash
cd $(dirname $0)

function update(){
	local is_running=`docker-compose ps --services --filter "status=running"`
	if [[ "$is_running" != "" ]]; then
		echo "services are still running!"
		echo "pulling new images..."
		docker-compose pull
		echo "pull complete. shutdown services..."
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
	"install") cp ./kuso-subdomain-adder-deploy.service /etc/systemd/system/ ;;
	"uninstall") systemctl stop kuso-subdomain-adder-deploy.service; rm /etc/systemd/system/kuso-subdomain-adder-deploy.service ;;
	"start") update; up > /dev/null ;;
	"stop") docker-compose down ;;
	"update") update ;;
esac
