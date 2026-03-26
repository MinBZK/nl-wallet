# OpenAPI Specifications

This directory contains the API documentation for the Wallet (as of this writing
2025-10-21, specifically, the internal API for the `verification_server` which
can be used to start a new session and get the disclosed attestations.

There are a couple of documents here:

  * [verification-server-components.openapi.yaml](../openapi/verification-server-components.openapi.yaml):
    An OpenAPIv3 components library which is for the verification server
    document below (you shouldn't open this in a Swagger/OpenAPI editor/preview
    on its own);
  * [verification-server-internal.openapi.yaml](../openapi/verification-server-internal.openapi.yaml):
    The private/internal API for the `verification_server`'s disclosure
    functionality;

You can open these OpenAPIv3 documents using Swagger UI and/or some facility in
your favorite editor (like Postman, or Redocly in Jetbrains editors which comes
as a part of the OpenAPI (Swagger) Editor Plugin).

To serve the OpenAPI specifications using a [Swagger UI][1] docker container:

```shell
cd nl-wallet
docker run --name swagger --detach --rm -p 8080:8080 \
-e URLS='[ { url: "openapi/verification-server-internal.openapi.yaml", name: "Verification Server Internal (requester) API" } ]' \
-e URLS_PRIMARY_NAME='Verification Server Internal (requester) API' \
-v "$PWD/wallet_docs/openapi":/usr/share/nginx/html/openapi \
docker.swagger.io/swaggerapi/swagger-ui
```

Then visit [http://localhost:8080](http://localhost:8080). The above docker
invocation executes the container in the background. To see the output of the
docker container, you can run `docker logs -f swagger`. To stop the container
(and remove it because we specified `--rm`), you can run `docker stop swagger`.

[1]: https://github.com/swagger-api/swagger-ui
