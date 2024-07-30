This directory contains the API documentation for the Wallet (as of this writing
2024-07-28, specifically, the `disclosure` API for the `wallet_server`'s public
and private endpoints).

There are a couple of documents here:

  * `wallet-disclosure-postman_collection.json`: A Postman collection containing
    API calling sessions for various scenarios, generated from recording a test
    run;
  * `wallet-disclosure-components.openapi.yaml`: An OpenAPIv3 components library
    which is included/used in the private and public API spec documents;
  * `wallet-disclosure-private.openapi.yaml`: The private/internal API for the
    wallet_server's disclosure functionality;
  * `wallet-disclosure-public.openapi.yaml`: The public/external API for the
    wallet_server's disclosure functionality;

You can "run" the last two OpenAPIv3 documents using Swagger and/or some
facility in your favorite editor (like Postmain, or Redocly in Jetbrains editors
which comes as a part of the OpenAPI (Swagger) Editor Plugin).
