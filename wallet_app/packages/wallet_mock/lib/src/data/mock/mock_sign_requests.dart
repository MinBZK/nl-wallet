import 'package:wallet_core/core.dart';

import '../../util/extension/string_extension.dart';
import '../model/requested_attribute.dart';
import '../model/sign_request.dart';
import 'mock_assets.dart';
import 'mock_organizations.dart';

final List<SignRequest> kSignRequests = [kRentalRequest];

final kRentalRequest = SignRequest(
  id: 'RENTAL_AGREEMENT',
  organization: kOrganizations[kHousingCorpId]!,
  trustProvider: Organization(
    legalName: 'Veilig Ondertekenen B.V.'.untranslated,
    displayName: 'Veilig Ondertekenen B.V.'.untranslated,
    category: 'Contracten'.untranslated,
    description: ''.untranslated,
    image: Image.asset(path: MockAssets.logo_sign_provider),
  ),
  document: const Document(
    title: 'Huurovereenkomst',
    fileName: '230110_Huurcontract_Bruijn.pdf',
    url: 'path/to/sample.pdf',
  ),
  requestedAttributes: [
    RequestedAttribute(key: 'mock.firstNames', label: 'Voornamen'),
    RequestedAttribute(key: 'mock.lastName', label: 'Achternaam'),
    RequestedAttribute(key: 'mock.birthDate', label: 'Geboortedatum'),
  ],
  policy: RequestPolicy(
    dataDeletionPossible: false,
    dataSharedWithThirdParties: false,
    dataStorageDurationInMinutes: BigInt.zero,
    policyUrl: 'https://example.org',
  ),
);
