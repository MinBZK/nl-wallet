import '../../../domain/model/data_attribute.dart';
import '../../../feature/verification/model/verification_request.dart';
import '../../../feature/verification/model/verifier.dart';
import '../../../feature/verification/model/verifier_policy.dart';
import 'verification_request_repository.dart';

class MockVerificationRepository implements VerificationRequestRepository {
  MockVerificationRepository();

  @override
  Future<VerificationRequest> getRequest(String sessionId) async {
    switch (sessionId) {
      case '1':
        return _kDuoSampleRequest;
      case '2':
        return _kLotterySampleRequest;
    }
    throw UnimplementedError('No mock usecase for id: $sessionId');
  }
}

const _kDuoSampleRequest = VerificationRequest(
  id: 1,
  verifier: Verifier(
      name: 'Dienst Uitvoering Onderwijs (DUO)',
      shortName: 'DUO',
      logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
      description:
          'Organisatie voor onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.'),
  attributes: [
    DataAttribute(type: 'Niveau', value: 'Master - WO'),
    DataAttribute(type: 'Onderwijsinstelling', value: 'Technische Universiteit Delft'),
    DataAttribute(type: 'Opleidingsnaam', value: 'Integrated Product Design'),
    DataAttribute(type: 'Verklaring Omtrent het Gedrag', value: 'Profiel 11, 12, 13'),
  ],
  policy: VerifierPolicy(
    storageDuration: Duration(days: 3 * 30),
    dataPurpose: 'Gegevens controle',
    privacyPolicyUrl: 'https://www.example.org',
    deletionCanBeRequested: true,
    dataIsShared: false,
  ),
);

const _kLotterySampleRequest = VerificationRequest(
  id: 2,
  verifier: Verifier(
    name: 'Nederlandse Staatsloterij',
    shortName: 'Staatsloterij',
    description:
        'Staatsloterij B.V. is een van de dochtervennootschappen van Nederlandse Loterij B.V.[1] De rechtsvoorganger Stichting Exploitatie Nederlandse Staatsloterij (SENS)[2] is in 1992 opgericht en heeft tot 2018 de Staatsloterij georganiseerd.',
    logoUrl: 'assets/non-free/images/logo_staatsloterij.png',
  ),
  attributes: [
    DataAttribute(type: 'Naam', value: 'John Doe'),
    DataAttribute(type: 'Email', value: 'john.doe@example.org'),
    DataAttribute(type: 'Telefoon', value: '+31623456789'),
    DataAttribute(type: 'Email', value: 'john.doe@example.org'),
  ],
  policy: VerifierPolicy(
    storageDuration: Duration(days: 30),
    dataPurpose: 'Gegevens controle',
    privacyPolicyUrl: 'https://www.example.org',
    deletionCanBeRequested: false,
    dataIsShared: true,
  ),
);
