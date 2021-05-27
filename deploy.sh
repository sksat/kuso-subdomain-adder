#!/bin/bash
cd $(dirname $0)

if [ ! -f .env ]; then
	echo ".env does not exist."
	exit 1
fi

source .env

function get_remote_img(){
	local tag=$1
	local user=sksat
	local repo=kuso-subdomain-adder

	echo -n "${tag} "
	curl --silent \
		--header "Accept: application/vnd.docker.distribution.manifest.v2+json" \
		--header "Authorization: Bearer" \
		"https://ghcr.io/v2/${user}/${repo}/manifests/${tag}" \
		| jq -r '.config.digest'
}

function get_local_img(){
	local tag=$1
	docker images --no-trunc --digests ghcr.io/sksat/kuso-subdomain-adder --format '{{.Tag}} {{.ID}}' | grep $tag
}

function update_repo(){
	local branch=`git symbolic-ref --short HEAD`
	local local_commit=`git rev-parse HEAD`
	local remote_commit=`git ls-remote origin ${branch} | awk '{print $1}'`

	echo "local:  ${local_commit}"
	echo "remote: ${remote_commit}"

	if [[ $local_commit != $remote_commit ]]; then
		echo "pulling repository..."
		git pull origin $branch
	fi
	echo ""
}

function update_image(){
	echo "update docker image..."

	local local_img=$(get_local_img $VERSION)
	local remote_img=$(get_remote_img $VERSION )

	echo "local:  $local_img"
	echo "remote: $remote_img"

	if [[ $local_img != $remote_img ]]; then
		echo "pulling new images..."
		docker-compose pull
	fi
	echo ""
}

function update(){
	local is_running=`docker-compose ps --services --filter "status=running"`

	update_repo
	update_image

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
