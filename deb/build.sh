#!/bin/bash

set -eu

NAME=pusher
BIN_PATH=$1
VERSION=$2
ARCH=$3
DEB_SRC="deb-${VERSION}-${ARCH}"
DEB_NAME="${NAME}_${VERSION}_${ARCH}.deb"

for bin in push-server push-send push-keygen; do
  install -Dm755 "${BIN_PATH}/${bin}" "${DEB_SRC}/usr/bin/${bin}"
  install -Dm644 "deb/${bin}.7" "${DEB_SRC}/usr/share/man/man7/${bin}.7"
  gzip -n --best "${DEB_SRC}/usr/share/man/man7/${bin}.7"
done

for bin in push-server push-send; do
  install -Dm600 "deb/${bin}.conf" "${DEB_SRC}/etc/${NAME}/${bin}.conf"
done
install -Dm644 "deb/push-server.service" "${DEB_SRC}/lib/systemd/system/push-server.service"
install -Dm644 "deb/push-send@.service" "${DEB_SRC}/lib/systemd/system/push-send@.service"
install -Dm644 "deb/push-sender.service" "${DEB_SRC}/lib/systemd/system/push-sender.service"
install -Dm644 assets/* -t "${DEB_SRC}/usr/share/${NAME}/assets"
install -Dm755 migrations/migrate.sh "${DEB_SRC}/usr/share/${NAME}/migrations/migrate.sh"
install -Dm644 migrations/*.sql -t "${DEB_SRC}/usr/share/${NAME}/migrations"

install -Dm644 README.md "${DEB_SRC}/usr/share/doc/${NAME}/README.md"
install -Dm644 deb/copyright "${DEB_SRC}/usr/share/doc/${NAME}/copyright"
sed 's/^/ /g' LICENSE >> "${DEB_SRC}/usr/share/doc/${NAME}/copyright"
install -Dm644 deb/changelog "${DEB_SRC}/usr/share/doc/${NAME}/changelog"
gzip -n --best "${DEB_SRC}/usr/share/doc/${NAME}/changelog"

install -Dm755 deb/postinst "${DEB_SRC}/DEBIAN/postinst"
install -Dm644 deb/conffiles "${DEB_SRC}/DEBIAN/conffiles"
install -Dm644 deb/control "${DEB_SRC}/DEBIAN/control"
sed -i "s/^Version:.*/Version: ${VERSION}/1" "${DEB_SRC}/DEBIAN/control"
sed -i "s/Architecture:.*/Architecture: ${ARCH}/1" "${DEB_SRC}/DEBIAN/control"

fakeroot dpkg-deb --build "${DEB_SRC}" "${DEB_NAME}"
lintian "${DEB_NAME}"

rm -r "${DEB_SRC}"
