ARG FROM_IMAGE_PREFIX
ARG TAG="latest"
FROM ${FROM_IMAGE_PREFIX}ci-android:${TAG}

# Dependency-Check
ENV DEPENDENCY_CHECK_HOME=/opt/dependency-check
ENV PATH=${PATH}:${DEPENDENCY_CHECK_HOME}/bin
COPY dependency-check.sh /dockerfiles/
RUN /dockerfiles/dependency-check.sh

# Sonar
ENV SONAR_HOME=/opt/sonar
ENV PATH=${PATH}:${SONAR_HOME}/bin
COPY sonar.sh /dockerfiles/
RUN /dockerfiles/sonar.sh

# CycloneDX
COPY cyclonedx.sh /dockerfiles/
RUN /dockerfiles/cyclonedx.sh

# OSV-Scanner
COPY osv-scanner.sh /dockerfiles/
RUN /dockerfiles/osv-scanner.sh

# Zap
ENV ZAP_HOME=/opt/zap
ENV PATH=${PATH}:${ZAP_HOME}
COPY zap.sh /dockerfiles/
RUN /dockerfiles/zap.sh
