import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';

void main() {
  test('Missing attribute equals works as expected', () {
    const attribute = MissingAttribute(key: 'key', label: {'nl': 'label'});
    const sameAttribute = MissingAttribute(key: 'key', label: {'nl': 'label'});
    expect(attribute, sameAttribute);
  });

  test('Missing attribute equals works as expected', () {
    final attribute = MissingAttribute.untranslated(key: 'key', label: 'label');
    final sameAttribute = MissingAttribute.untranslated(key: 'key', label: 'label');
    expect(attribute, sameAttribute);
  });

  test('Missing attribute !equals works as expected', () {
    final attribute = MissingAttribute.untranslated(key: 'key', label: 'label');
    final otherLabel = MissingAttribute.untranslated(key: 'key', label: 'other');
    final otherKey = MissingAttribute.untranslated(key: 'key2', label: 'label');
    expect(attribute == otherLabel, isFalse);
    expect(attribute == otherKey, isFalse);
  });
}
