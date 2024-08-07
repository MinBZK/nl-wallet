import 'package:flutter/widgets.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

class DeviceUtils {
  DeviceUtils._();

  static const pixel2Portrait =
      Device(size: Size(411, 683), name: 'pixel2_portrait', textScale: 1, devicePixelRatio: 2.625);
  static const pixel2PortraitLarge =
      Device(size: Size(346, 566), name: 'pixel2_portrait_large', textScale: 1.3, devicePixelRatio: 3.125);
  static const pixel2Landscape =
      Device(size: Size(667, 375), name: 'pixel2_landscape', textScale: 1, devicePixelRatio: 2.625);
  static const pixel2LandscapeLarge =
      Device(size: Size(566, 346), name: 'pixel2_landscape_large', textScale: 1.3, devicePixelRatio: 3.125);

  static final _accessibilityDevices = [
    pixel2Portrait,
    pixel2PortraitLarge,
    pixel2Landscape,
    pixel2LandscapeLarge,
  ];

  static DeviceBuilder get deviceBuilder =>
      DeviceBuilder()..overrideDevicesForAllScenarios(devices: _accessibilityDevices);

  static DeviceBuilder get deviceBuilderWithPrimaryScrollController =>
      DeviceBuilder(wrap: (child) => PrimaryScrollController(controller: ScrollController(), child: child))
        ..overrideDevicesForAllScenarios(devices: _accessibilityDevices);
}
