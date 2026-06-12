import 'package:wallet_core/core.dart';

import '../../../mock.dart';
import 'mock_attributes.dart';
import 'mock_organizations.dart';

const _kRijksLogoAsset = 'assets/non-free/logos/card_rijksoverheid.png';
const _kWhite = '#FFFFFF';
const _kDarkBlue = '#152A62';
const _kEducationSummary = '{{mock_education}}';
const _kDrivingLicenseSummary = '{{mock_drivingLicenseCategories}}';

const kValidityStatus = ValidityStatus_Valid(validUntil: null);

final kPidAttestations = [
  AttestationPresentation(
    identity: const AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.pid,
    format: Format.SdJwt,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Personal data',
      dutchTitle: 'Persoons­gegevens',
      englishSummary: '{{mock_firstNames}}',
      dutchSummary: '{{mock_firstNames}}',
      logoAsset: _kRijksLogoAsset,
      textColor: _kDarkBlue,
    ),
    issuer: kOrganizations[kRvigId]!,
    validityStatus: kValidityStatus,
    attributes: kMockPidAttestationAttributes,
  ),
  AttestationPresentation(
    identity: const AttestationIdentity_Ephemeral(),
    attestationType: MockAttestationTypes.address,
    format: Format.SdJwt,
    displayMetadata: _buildDisplayMetaData(
      englishTitle: 'Residential address',
      dutchTitle: 'Woonadres',
      englishSummary: '{{mock_city}}',
      dutchSummary: '{{mock_city}}',
      logoAsset: _kRijksLogoAsset,
      textColor: _kWhite,
    ),
    issuer: kOrganizations[kRvigId]!,
    validityStatus: kValidityStatus,
    attributes: kMockAddressAttestationAttributes,
  ),
];

final kDiplomaAttestation = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.bscDiploma,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'BSc. Diploma',
    dutchTitle: 'BSc. Diploma',
    englishSummary: _kEducationSummary,
    dutchSummary: _kEducationSummary,
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
  ),
  validityStatus: kValidityStatus,
  attributes: kMockDiplomaAttestationAttributes,
  issuer: kOrganizations[kDuoId]!,
);

final kMockMasterDiplomaWalletCard = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.mscDiploma,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'MSc. Diploma',
    dutchTitle: 'MSc. Diploma',
    englishSummary: _kEducationSummary,
    dutchSummary: _kEducationSummary,
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
  ),
  validityStatus: kValidityStatus,
  attributes: kMockMasterDiplomaDataAttributes,
  issuer: kOrganizations[kDuoId]!,
);

final kMockDrivingLicenseWalletCard = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.drivingLicense,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: _kDrivingLicenseSummary,
    dutchSummary: _kDrivingLicenseSummary,
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: _kDarkBlue,
  ),
  validityStatus: kValidityStatus,
  attributes: kMockDrivingLicenseDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockDrivingLicenseRenewedWalletCard = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.drivingLicense,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Driving License',
    dutchTitle: 'Rijbewijs',
    englishSummary: _kDrivingLicenseSummary,
    dutchSummary: _kDrivingLicenseSummary,
    logoAsset: 'assets/non-free/logos/nl_driving_license.png',
    textColor: _kDarkBlue,
  ),
  validityStatus: kValidityStatus,
  attributes: kMockDrivingLicenseRenewedDataAttributes,
  issuer: kOrganizations[kRdwId]!,
);

final kMockHealthInsuranceWalletCard = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.healthInsurance,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'European Health Insurance Card',
    dutchTitle: 'Europese gezondheidskaart',
    englishSummary: '{{mock_healthIssuerId}}',
    dutchSummary: '{{mock_healthIssuerId}}',
    logoAsset: 'assets/non-free/logos/nl_health_insurance.png',
    textColor: _kWhite,
  ),
  validityStatus: kValidityStatus,
  attributes: kMockHealthInsuranceDataAttributes,
  issuer: kOrganizations[kHealthInsuranceId]!,
);

final kMockVOGWalletCard = AttestationPresentation(
  identity: const AttestationIdentity.ephemeral(),
  attestationType: MockAttestationTypes.certificateOfConduct,
  format: Format.SdJwt,
  displayMetadata: _buildDisplayMetaData(
    englishTitle: 'Certificate of Conduct',
    dutchTitle: 'Verklaring Omtrent het Gedrag',
    englishSummary: 'Valid through {{mock_expiryDate}}',
    dutchSummary: 'Geldig tot {{mock_expiryDate}}',
    logoAsset: _kRijksLogoAsset,
    textColor: _kWhite,
  ),
  validityStatus: kValidityStatus,
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
      locale: 'en',
      name: englishTitle,
      summary: englishSummary,
      rendering: RenderingMetadata.simple(
        logo: logoAsset == null
            ? null
            : ImageWithMetadata(
                image: Image.asset(path: logoAsset),
                altText: '',
              ),
        textColor: textColor,
        backgroundColor: bgColor,
      ),
    ),
    DisplayMetadata(
      locale: 'nl',
      name: dutchTitle,
      summary: dutchSummary,
      rendering: RenderingMetadata.simple(
        logo: logoAsset == null
            ? null
            : ImageWithMetadata(
                image: Image.asset(path: logoAsset),
                altText: '',
              ),
        textColor: textColor,
        backgroundColor: bgColor,
      ),
    ),
  ];
}
