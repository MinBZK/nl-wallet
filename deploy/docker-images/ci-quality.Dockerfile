ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-android:${TAG}

# Dependency-Check
ENV DEPENDENCY_CHECK_HOME=/opt/dependency-check
ENV PATH=${PATH}:${DEPENDENCY_CHECK_HOME}/bin
COPY dependency-check.sh /tmp/
RUN /tmp/dependency-check.sh

# Sonar
ENV SONAR_HOME=/opt/sonar
ENV PATH=${PATH}:${SONAR_HOME}/bin
COPY sonar.sh /tmp/
RUN /tmp/sonar.sh

# CycloneDX
COPY cyclonedx.sh /tmp/
RUN /tmp/cyclonedx.sh

# OSV-Scanner
COPY osv-scanner.sh /tmp/
RUN /tmp/osv-scanner.sh

# Zap
ENV ZAP_HOME=/opt/zap
ENV PATH=${PATH}:${ZAP_HOME}
COPY zap.sh /tmp/
RUN /tmp/zap.sh

# Cleanup
RUN rm -rf /tmp/*
