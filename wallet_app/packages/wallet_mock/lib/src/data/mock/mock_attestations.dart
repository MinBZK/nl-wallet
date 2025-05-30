import 'package:wallet_core/core.dart';

import '../../../mock.dart';
import 'mock_attributes.dart';
import 'mock_organizations.dart';

final kPidAttestations = [
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'pid'),
    attestationType: MockConstants.pidDocType,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Personal data',
      dutchTitle: 'Persoons­gegevens',
      englishSummary: '{{mock_firstNames}}',
      dutchSummary: '{{mock_firstNames}}',
      logoAsset: 'assets/non-free/logos/card_rijksoverheid.png',
      textColor: '#152A62',
    ),
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockPidAttestationAttributes,
  ),
  Attestation(
    identity: AttestationIdentity_Fixed(id: 'address'),
    attestationType: MockConstants.addressDocType,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Residential address',
      dutchTitle: 'Woonadres',
      englishSummary: '{{mock_city}}',
      dutchSummary: '{{mock_city}}',
      logoAsset: 'assets/non-free/logos/card_rijksoverheid.png',
      textColor: '#FFFFFF',
    ),
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockAddressAttestationAttributes,
  ),
];

const _kDiplomaId = 'DIPLOMA_1';
final kDiplomaAttestation = Attestation(
  attestationType: _kDiplomaId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'BSc. Diploma',
    dutchTitle: 'BSc. Diploma',
    englishSummary: '{{mock_education}}',
    dutchSummary: '{{mock_education}}',
    logoAsset: 'assets/non-free/logos/card_rijksoverheid.png',
    textColor: '#FFFFFF',
  ),
  attributes: kMockDiplomaAttestationAttributes,
  issuer: kOrganizations[kDuoId]!,
);

const _kMasterDiplomaId = 'DIPLOMA_2';
final kMockMasterDiplomaWalletCard = Attestation(
  attestationType: _kMasterDiplomaId,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'MSc. Diploma',
    dutchTitle: 'MSc. Diploma',
    englishSummary: '{{mock_education}}',
    dutchSummary: '{{mock_education}}',
    logoAsset: 'assets/non-free/logos/card_rijksoverheid.png',
    textColor: '#FFFFFF',
  ),
  attributes: kMockMasterDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

final kMockDrivingLicenseWalletCard = Attestation(
  attestationType: MockConstants.drivingLicenseDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: '{{mock_drivingLicenseCategories}}',
    dutchSummary: '{{mock_drivingLicenseCategories}}',
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: '#152A62',
  ),
  attributes: kMockDrivingLicenseDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockDrivingLicenseRenewedWalletCard = Attestation(
  attestationType: MockConstants.drivingLicenseDocType,
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: '{{mock_drivingLicenseCategories}}',
    dutchSummary: '{{mock_drivingLicenseCategories}}',
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: '#152A62',
  ),
  attributes: kMockDrivingLicenseRenewedDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockHealthInsuranceWalletCard = Attestation(
  attestationType: 'HEALTH_INSURANCE',
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'European Health Insurance Card',
    dutchTitle: 'Europese gezondheidskaart',
    englishSummary: '{{mock_healthIssuerId}}',
    dutchSummary: '{{mock_healthIssuerId}}',
    logoAsset: 'assets/non-free/logos/nl_health_insurance.png',
    textColor: '#FFFFFF',
  ),
  attributes: kMockHealthInsuranceDataAttributes,
  issuer: kOrganizations[kHealthInsuranceId]!,
);

final kMockVOGWalletCard = Attestation(
  attestationType: 'VOG',
  identity: AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Certificate of Conduct',
    dutchTitle: 'Verklaring Omtrent het Gedrag',
    englishSummary: 'Valid through {{mock_expiryDate}}',
    dutchSummary: 'Geldig tot {{mock_expiryDate}}',
    logoAsset: 'assets/non-free/logos/card_rijksoverheid.png',
    textColor: '#FFFFFF',
  ),
  attributes: kMockVOGDataAttributes,
  issuer: kOrganizations[kRvigId]!,
);

List<DisplayMetadata> _buildDisplayMetaData({
  required String englishTitle,
  required String dutchTitle,
  String? englishSummary,
  String? dutchSummary,
  String? logoAsset,
  String? textColor,
  String? bgColor,
}) {
  return [
    DisplayMetadata(
      lang: 'en',
      name: englishTitle,
      summary: englishSummary,
      rendering: RenderingMetadata.simple(
        logo: logoAsset == null ? null : ImageWithMetadata(image: Image.asset(path: logoAsset), altText: ''),
        textColor: textColor,
        backgroundColor: bgColor,
      ),
    ),
    DisplayMetadata(
      lang: 'nl',
      name: dutchTitle,
      summary: dutchSummary,
      rendering: RenderingMetadata.simple(
        logo: logoAsset == null ? null : ImageWithMetadata(image: Image.asset(path: logoAsset), altText: ''),
        textColor: textColor,
        backgroundColor: bgColor,
      ),
    ),
  ];
}
