ARG DOCKER_HUB_PROXY
# trixie-20250721-slim
FROM ${DOCKER_HUB_PROXY}library/debian@sha256:cc92da07b99dd5c078cb5583fdb4ba639c7c9c14eb78508a2be285ca67cc738a

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
