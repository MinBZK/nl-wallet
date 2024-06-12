import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/qr/edi_qr_code.dart';

void main() {
  setUp(() {});

  group('fromJson', () {
    test('json with issue type should return issuance type', () {
      final Map<String, dynamic> input = {'id': '', 'type': 'issue'};
      expect(EdiQrCode.fromJson(input).type, EdiQrType.issuance);
    });

    test('json with verify type should return disclosure type', () {
      final Map<String, dynamic> input = {'id': '', 'type': 'verify'};
      expect(EdiQrCode.fromJson(input).type, EdiQrType.disclosure);
    });

    test('json with sign type should return sign type', () {
      final Map<String, dynamic> input = {'id': '', 'type': 'sign'};
      expect(EdiQrCode.fromJson(input).type, EdiQrType.sign);
    });
  });
}
