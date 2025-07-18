import 'package:wallet_core/core.dart';

import '../../../mock.dart';
import 'mock_attributes.dart';
import 'mock_organizations.dart';

const _kRijksLogoAsset = 'assets/non-free/logos/card_rijksoverheid.png';
const _kWhite = '#FFFFFF';
const _kDarkBlue = '#152A62';
const _kEducationSummary = '{{mock_education}}';
const _kDrivingLicenseSummary = '{{mock_drivingLicenseCategories}}';

final kPidAttestations = [
  AttestationPresentation(
    identity: const AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.pid,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Personal data',
      dutchTitle: 'Persoons­gegevens',
      englishSummary: '{{mock_firstNames}}',
      dutchSummary: '{{mock_firstNames}}',
      logoAsset: _kRijksLogoAsset,
      textColor: _kDarkBlue,
    ),
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockPidAttestationAttributes,
  ),
  AttestationPresentation(
    identity: const AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.address,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Residential address',
      dutchTitle: 'Woonadres',
      englishSummary: '{{mock_city}}',
      dutchSummary: '{{mock_city}}',
      logoAsset: _kRijksLogoAsset,
      textColor: _kWhite,
    ),
    issuer: kOrganizations[kRvigId]!,
    attributes: kMockAddressAttestationAttributes,
  ),
];

final kDiplomaAttestation = AttestationPresentation(
  attestationType: MockAttestationTypes.bscDiploma,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'BSc. Diploma',
    dutchTitle: 'BSc. Diploma',
    englishSummary: _kEducationSummary,
    dutchSummary: _kEducationSummary,
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
  ),
  attributes: kMockDiplomaAttestationAttributes,
  issuer: kOrganizations[kDuoId]!,
);

final kMockMasterDiplomaWalletCard = AttestationPresentation(
  attestationType: MockAttestationTypes.mscDiploma,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'MSc. Diploma',
    dutchTitle: 'MSc. Diploma',
    englishSummary: _kEducationSummary,
    dutchSummary: _kEducationSummary,
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
  ),
  attributes: kMockMasterDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

final kMockDrivingLicenseWalletCard = AttestationPresentation(
  attestationType: MockAttestationTypes.drivingLicense,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: _kDrivingLicenseSummary,
    dutchSummary: _kDrivingLicenseSummary,
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: _kDarkBlue,
  ),
  attributes: kMockDrivingLicenseDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockDrivingLicenseRenewedWalletCard = AttestationPresentation(
  attestationType: MockAttestationTypes.drivingLicense,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: _kDrivingLicenseSummary,
    dutchSummary: _kDrivingLicenseSummary,
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: _kDarkBlue,
  ),
  attributes: kMockDrivingLicenseRenewedDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockHealthInsuranceWalletCard = AttestationPresentation(
  attestationType: MockAttestationTypes.healthInsurance,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'European Health Insurance Card',
    dutchTitle: 'Europese gezondheidskaart',
    englishSummary: '{{mock_healthIssuerId}}',
    dutchSummary: '{{mock_healthIssuerId}}',
    logoAsset: 'assets/non-free/logos/nl_health_insurance.png',
    textColor: _kWhite,
  ),
  attributes: kMockHealthInsuranceDataAttributes,
  issuer: kOrganizations[kHealthInsuranceId]!,
);

final kMockVOGWalletCard = AttestationPresentation(
  attestationType: MockAttestationTypes.certificateOfConduct,
  identity: const AttestationIdentity.ephemeral(),
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Certificate of Conduct',
    dutchTitle: 'Verklaring Omtrent het Gedrag',
    englishSummary: 'Valid through {{mock_expiryDate}}',
    dutchSummary: 'Geldig tot {{mock_expiryDate}}',
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
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
