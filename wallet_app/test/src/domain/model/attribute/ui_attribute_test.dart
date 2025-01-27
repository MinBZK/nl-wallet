import 'package:flutter/material.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';

void main() {
  test('UiAttribute key should throw', () {
    const attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {'nl': 'test'},
    );

    expect(() => attribute.key, throwsA(isA<UnsupportedError>()));
  });

  test('UiAttribute equals works as expected', () {
    const attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {'nl': 'test'},
    );

    const identicalAttribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {'nl': 'test'},
    );

    expect(attribute == identicalAttribute, isTrue);
  });

  test('UiAttribute !equals works as expected', () {
    const attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {'nl': 'test'},
    );

    const otherValue = UiAttribute(
      value: StringValue('value!'),
      icon: Icons.connected_tv_sharp,
      label: {'nl': 'test'},
    );

    const otherIcon = UiAttribute(
      value: StringValue('value'),
      icon: Icons.factory,
      label: {'nl': 'test'},
    );

    const otherLabel = UiAttribute(
      value: StringValue('value'),
      icon: Icons.factory,
      label: {'en': 'test'},
    );

    expect(attribute == otherValue, isFalse);
    expect(attribute == otherIcon, isFalse);
    expect(attribute == otherLabel, isFalse);
  });
}
