import 'dart:io';

import 'package:arb_utils/arb_utils.dart';

const _kAppEnArbFileName = 'lib/src/localization/app_en.arb';
const _kAppNlArbFileName = 'lib/src/localization/app_nl.arb';

void main() {
  //Process English translations
  final enArbFile = File(_kAppEnArbFileName);
  final enArbContents = enArbFile.readAsStringSync();
  enArbFile.writeAsStringSync(sortARB(enArbContents));

  //Process Dutch translations
  final nlArbFile = File(_kAppNlArbFileName);
  final nlArbContents = nlArbFile.readAsStringSync();
  nlArbFile.writeAsStringSync(sortARB(nlArbContents));
}
