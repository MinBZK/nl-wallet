ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-base:${TAG}

COPY ansible.sh /tmp/
RUN sudo -E -H -u wallet -- sh -c 'cd $HOME && /tmp/ansible.sh'

# Cleanup
RUN rm -rf /tmp/*
