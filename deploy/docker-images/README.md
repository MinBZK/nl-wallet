# Docker images

This folder contains Dockerfiles that are used as build images for the CI.

All external dependencies are locked to hash as much as possible. All APT
repository signatures are also locked to hash. This is because apt-get update
and apt-get upgrade are used when building. Locking down to the docker SHA-256
hash is insufficent.

## CI image

The CI image is layered due to restrictions of Standard Platform. It cannot
build one great image and it cannot handle large images from the nodes either.
It is important to keep the CI image layered (in contrast with executing same
scripts) in order to save disk space in both Harbor and the Kubernetes nodes
that pull the image. Likewise the pipeline needs to use the same tag to exploit
this feature.

The layers can be used independently which saves pull time on the node and
failures compared to one big image. The order of the layers is somewhat
arbitrary:

- The base layer contains common relatively small tooling.
- Second is the web layer which is also small and has a playwright descendant
  that doesn't need the other layers.
- The rust, flutter, android layer can be reshuffled and are separated because
  they are pretty big layers.
- The quality layer is on top as most tooling needs execute commands to get more
  information (`flutter pub get`, `npm ci`).

Note that small total images is explicitly a non-goal. We would like to have a
single image because it makes CI life easy. It doesn't really matter how the
layers are built up in. In Docker there isn't a difference in pulling or storage
size (only in starting a bit) The only reason we name the layers is that we can
use these because SP has problems with pulling a big image.
