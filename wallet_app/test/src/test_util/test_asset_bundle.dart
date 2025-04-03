// Inspired by golden_toolkit:
// Copyright 2019-2020 eBay Inc.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/BSD-3-Clause

import 'dart:convert';

import 'package:flutter/services.dart';

class TestAssetBundle extends CachingAssetBundle {
  @override
  Future<String> loadString(String key, {bool cache = true}) async {
    // overriding this method to avoid limit of 10KB per asset
    final data = await load(key);
    return utf8.decode(data.buffer.asUint8List());
  }

  @override
  Future<ByteData> load(String key) async => rootBundle.load(key);
}
