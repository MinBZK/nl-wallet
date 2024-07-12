import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/document.dart';

void main() {
  test('Verify document equals', () {
    const doc = Document(
      title: 'Title',
      fileName: 'docs/agreement.pdf',
      url: 'https://example.org/agreement.pdf',
    );
    const docIdentical = Document(
      title: 'Title',
      fileName: 'docs/agreement.pdf',
      url: 'https://example.org/agreement.pdf',
    );
    const docDifferent = Document(
      title: 'Other document title',
      fileName: 'docs/agreement.pdf',
      url: 'https://example.org/agreement.pdf',
    );
    expect(doc, docIdentical);
    expect(doc, isNot(equals(docDifferent)));
  });
}
