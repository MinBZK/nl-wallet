# OpenAPI Specifications

This directory contains the API documentation for the Wallet (as of this writing
2025-10-21, specifically, the disclosure API for the `verification_server`'s
public and private endpoints, and the `issuance_server`'s issuance and
disclosure-based issuance endpoints).

There are a couple of documents here:

  * [wallet-components.openapi.yaml](../openapi/wallet-components.openapi.yaml):
    An OpenAPIv3 components library which are used by our API specification
    documents below (you shouldn't open this in a Swagger/OpenAPI editor/preview
    on its own);
  * [wallet-disclosure-private.openapi.yaml](../openapi/wallet-disclosure-private.openapi.yaml):
    The private/internal API for the `verification_server`'s disclosure
    functionality;
  * [wallet-disclosure-public.openapi.yaml](../openapi/wallet-disclosure-public.openapi.yaml):
    The public/external API for the `verification_server`'s disclosure
    functionality;
  * [wallet-issuance.openapi.yaml](../openapi/wallet-issuance.openapi.yaml):
    The API for the `issuance_server`'s issuance and disclosure-based issuance
    functionality;

You can open these OpenAPIv3 documents using Swagger UI and/or some facility in
your favorite editor (like Postman, or Redocly in Jetbrains editors which comes
as a part of the OpenAPI (Swagger) Editor Plugin).

To serve the OpenAPI specifications using a [Swagger UI][1] docker container:

```shell
cd nl-wallet
docker run --name swagger --detach --rm -p 8080:8080 \
-e URLS='[ { url: "openapi/wallet-disclosure-private.openapi.yaml", name: "Disclosure Private (requester) API" }, { url: "openapi/wallet-disclosure-public.openapi.yaml", name: "Disclosure Public (wallet) API" },  { url: "openapi/wallet-issuance.openapi.yaml", name: "Issuer API" } ]' \
-e URLS_PRIMARY_NAME='Disclosure Private (requester) API' \
-v "$PWD/wallet_docs/openapi":/usr/share/nginx/html/openapi \
docker.swagger.io/swaggerapi/swagger-ui
```

Then visit [http://localhost:8080](http://localhost:8080). The above docker
invocation executes the container in the background. To see the output of the
docker container, you can run `docker logs -f swagger`. To stop the container
(and remove it because we specified `--rm`), you can run `docker stop swagger`.

[1]: https://github.com/swagger-api/swagger-ui
