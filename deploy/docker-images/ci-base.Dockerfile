ARG DOCKER_HUB_PROXY
# bookworm-20250630-slim
FROM ${DOCKER_HUB_PROXY}library/debian@sha256:6ac2c08566499cc2415926653cf2ed7c3aedac445675a013cc09469c9e118fdd

COPY apt.sh /dockerfiles/
RUN /dockerfiles/apt.sh

# Update and upgrade to the latest and greatest
RUN apt-get update && apt-get upgrade -y

# Add wallet user with same uid as gitlab-runner and
RUN useradd -u 1001 -m -d /wallet -s /bin/bash wallet \
 # make /opt wallet group writable
 && chgrp wallet /opt && chmod g+w /opt

WORKDIR /wallet

# Standard utils
COPY utils.sh /dockerfiles/
RUN /dockerfiles/utils.sh

# Python
COPY python.sh /dockerfiles/
RUN /dockerfiles/python.sh
ENV PATH=${PATH}:/wallet/.local/bin

# Ruby
COPY ruby.sh /dockerfiles/
RUN /dockerfiles/ruby.sh

# Java
COPY java.sh /dockerfiles/
RUN /dockerfiles/java.sh

# Change home dir of root to make image workable as root
RUN sed -i'' '/^root:/ { s#:/root:#:/wallet:# }' /etc/passwd

# No `USER wallet` as gitlab-runners are free to choose build path but will honor the USER command.
# This way the build image will also be usable when executed as root on a standard runner.
