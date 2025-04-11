import 'package:flutter/material.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';

void main() {
  test('UiAttribute key should throw', () {
    final attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {Locale('nl'): 'test'},
    );

    expect(() => attribute.key, throwsA(isA<UnsupportedError>()));
  });

  test('UiAttribute equals works as expected', () {
    final attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {Locale('nl'): 'test'},
    );

    final identicalAttribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {Locale('nl'): 'test'},
    );

    expect(attribute == identicalAttribute, isTrue);
  });

  test('UiAttribute !equals works as expected', () {
    final attribute = UiAttribute(
      value: StringValue('value'),
      icon: Icons.connected_tv_sharp,
      label: {Locale('nl'): 'test'},
    );

    final otherValue = UiAttribute(
      value: StringValue('value!'),
      icon: Icons.connected_tv_sharp,
      label: {Locale('nl'): 'test'},
    );

    final otherIcon = UiAttribute(
      value: StringValue('value'),
      icon: Icons.factory,
      label: {Locale('nl'): 'test'},
    );

    final otherLabel = UiAttribute(
      value: StringValue('value'),
      icon: Icons.factory,
      label: {Locale('en'): 'test'},
    );

    expect(attribute == otherValue, isFalse);
    expect(attribute == otherIcon, isFalse);
    expect(attribute == otherLabel, isFalse);
  });
}
