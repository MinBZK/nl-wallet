ARG DOCKER_HUB_PROXY
# trixie-20260518-slim
FROM ${DOCKER_HUB_PROXY}library/debian@sha256:b6e2a152f22a40ff69d92cb397223c906017e1391a73c952b588e51af8883bf8

COPY apt.sh /tmp/
RUN /tmp/apt.sh

# Update and upgrade to the latest and greatest
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y

# Add wallet user with same uid as gitlab-runner and
RUN useradd -u 1001 -m -d /wallet -s /bin/bash wallet \
 # make /opt wallet group writable
 && chgrp wallet /opt && chmod g+w /opt

WORKDIR /wallet

# Standard utils
COPY utils.sh /tmp/
RUN /tmp/utils.sh

# Python
COPY python.sh /tmp/
RUN /tmp/python.sh
ENV PATH=${PATH}:/wallet/.local/bin

# Ruby
COPY ruby.sh /tmp/
RUN /tmp/ruby.sh

# Java
COPY java.sh /tmp/
RUN /tmp/java.sh

# Change home dir of root to make image workable as root
RUN sed -i'' '/^root:/ { s#:/root:#:/wallet:# }' /etc/passwd

# Cleanup
RUN rm -rf /tmp/*

# No `USER wallet` as gitlab-runners are free to choose build path but will honor the USER command.
# This way the build image will also be usable when executed as root on a standard runner.
