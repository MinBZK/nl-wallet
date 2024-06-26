import 'package:flutter/cupertino.dart';

extension ScrollControllerExtensions on ScrollController {
  double maxScrollExtent({double fallback = 0}) {
    return (hasClients && position.hasContentDimensions) ? position.maxScrollExtent : fallback;
  }
}
