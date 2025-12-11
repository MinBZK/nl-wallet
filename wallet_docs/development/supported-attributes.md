# Supported attributes

In this document you'll find a collection of claim paths within a number of
verifiable credential types we support. These claim paths are used to indicate
what values you're interested in, for example within the `authorizedAttributes`
object in our `reader_auth.json` document.

<div class="admonition note"><p class="title">These tables are generated</p>
<p>The below tables are generated using our `supported-attributes.sh` script,
which parses our `scripts/devenv/eudi:*.json` documents. If you suspect the
claim path attributes you're looking at might be out-of-date, you can invoke
`supported-attributes.sh` and make sure you're looking at the latest we support.
</p></div>

## Claims in eudi:pid:1

| Claim Path    | Label         | Description                                       | Language |
| ------------- | ------------- | ------------------------------------------------- | -------- |
| family_name   | Name          | Family name of the person, including any prefixes | en-US    |
| given_name    | First name    | First name of the person                          | en-US    |
| birthdate     | Birth date    | Birth date of the person                          | en-US    |
| age_over_18   | Over 18       | Whether the person is over 18                     | en-US    |
| nationalities | Nationalities | List of nationalities of the person               | en-US    |

## Claims in eudi:pid-address:1

| Claim Path             | Label        | Description                 | Language |
| ---------------------- | ------------ | --------------------------- | -------- |
| address.country        | Country      | Country of the address      | en-US    |
| address.locality       | City         | City of the address         | en-US    |
| address.postal_code    | Postal code  | Postal code of the address  | en-US    |
| address.street_address | Street       | Street of the address       | en-US    |
| address.house_number   | House number | House number of the address | en-US    |

## Claims in eudi:pid-address:nl:1

| Claim Path             | Label        | Description                 | Language |
| ---------------------- | ------------ | --------------------------- | -------- |
| address.country        | Country      | Country of the address      | en-US    |
| address.country        | Land         | Land van het adres          | nl-NL    |
| address.house_number   | House number | House number of the address | en-US    |
| address.house_number   | Huisnummer   | Huisnummer van het adres    | nl-NL    |
| address.locality       | City         | City of the address         | en-US    |
| address.locality       | Stad         | Stad van het adres          | nl-NL    |
| address.postal_code    | Postal code  | Postal code of the address  | en-US    |
| address.postal_code    | Postcode     | Postcode van het adres      | nl-NL    |
| address.street_address | Straatnaam   | Straatnaam van het adres    | nl-NL    |
| address.street_address | Street       | Street of the address       | en-US    |

## Claims in eudi:pid:nl:1

| Claim Path    | Label           | Description                                       | Language |
| ------------- | --------------- | ------------------------------------------------- | -------- |
| age_over_18   | 18+             | Of de persoon 18+ is                              | nl-NL    |
| age_over_18   | Over 18         | Whether the person is over 18                     | en-US    |
| birthdate     | Geboortedatum   | Geboortedatum van de persoon                      | nl-NL    |
| birthdate     | Birth date      | Birth date of the person                          | en-US    |
| bsn           | BSN             | BSN van de persoon                                | nl-NL    |
| bsn           | BSN             | BSN of the person                                 | en-US    |
| family_name   | Achternaam      | Achternaam van de persoon, inclusief voorvoegsels | nl-NL    |
| family_name   | Name            | Family name of the person, including any prefixes | en-US    |
| given_name    | Voornaam        | Voornaam van de persoon                           | nl-NL    |
| given_name    | First name      | First name of the person                          | en-US    |
| nationalities | Nationaliteiten | Lijst van nationaliteiten van de persoon          | nl-NL    |
| nationalities | Nationalities   | List of nationalities of the person               | en-US    |
| recovery_code | Herstelcode     | Herstelcode van de persoon                        | nl-NL    |
| recovery_code | Recovery code   | Recovery code of the person                       | en-US    |
