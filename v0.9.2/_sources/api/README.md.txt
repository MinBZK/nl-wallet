# Open Api Specifications

This directory contains the API documentation for the Wallet (as of this writing
2025-02-24, specifically, the disclosure API for the `verification_server`'s public
and private endpoints).

There are a couple of documents here:

  * [wallet-disclosure-components.openapi.yaml](../_static/openapi/wallet-disclosure-components.openapi.yaml): An OpenAPIv3 components library
    which is included/used in the private and public API spec documents below.
    You shouldn't open this in a Swagger/OpenAPI editor/preview on its own;
  * [wallet-disclosure-private.openapi.yaml](../_static/openapi/wallet-disclosure-private.openapi.yaml): The private/internal API for the
    `verification_server`'s disclosure functionality;
  * [wallet-disclosure-public.openapi.yaml](../_static/openapi/wallet-disclosure-public.openapi.yaml): The public/external API for the
    `verification_server`'s disclosure functionality;

You can "run" the last two OpenAPIv3 documents using Swagger and/or some
facility in your favorite editor (like Postman, or Redocly in Jetbrains editors
which comes as a part of the OpenAPI (Swagger) Editor Plugin).
