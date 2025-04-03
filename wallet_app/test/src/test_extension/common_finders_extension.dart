import 'package:flutter_test/flutter_test.dart';

extension CommonFindersExtensions on CommonFinders {
  Finder get root => find.byElementPredicate((w) => true).first;
}
