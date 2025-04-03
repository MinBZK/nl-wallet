// Inspired by golden_toolkit:
// Copyright 2019-2020 eBay Inc.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/BSD-3-Clause

import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

class FontUtils {
  FontUtils._();

  static Future<void> loadAppFonts() async {
    TestWidgetsFlutterBinding.ensureInitialized();
    final fontManifest = await rootBundle.loadStructuredData<Iterable<dynamic>>(
      'FontManifest.json',
      (string) async => json.decode(string),
    );

    for (final Map<String, dynamic> font in fontManifest) {
      final fontLoader = FontLoader(_derivedFontFamily(font));
      for (final Map<String, dynamic> fontType in font['fonts']) {
        fontLoader.addFont(rootBundle.load(fontType['asset']));
      }
      await fontLoader.load();
    }
  }

  static String _derivedFontFamily(Map<String, dynamic> fontDefinition) {
    if (!fontDefinition.containsKey('family')) {
      return '';
    }

    final String fontFamily = fontDefinition['family'];

    if (_overridableFonts.contains(fontFamily)) {
      return fontFamily;
    }

    if (fontFamily.startsWith('packages/')) {
      final fontFamilyName = fontFamily.split('/').last;
      if (_overridableFonts.any((font) => font == fontFamilyName)) {
        return fontFamilyName;
      }
    } else {
      for (final Map<String, dynamic> fontType in fontDefinition['fonts']) {
        final String? asset = fontType['asset'];
        if (asset != null && asset.startsWith('packages')) {
          final packageName = asset.split('/')[1];
          return 'packages/$packageName/$fontFamily';
        }
      }
    }
    return fontFamily;
  }

  static const List<String> _overridableFonts = [
    'Roboto',
    '.SF UI Display',
    '.SF UI Text',
    '.SF Pro Text',
    '.SF Pro Display',
  ];
}
