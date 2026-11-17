import 'package:cupertino_ui/cupertino_ui.dart';

extension ScrollControllerExtensions on ScrollController {
  double maxScrollExtent({double fallback = 0}) {
    return (hasClients && positions.first.hasContentDimensions) ? positions.first.maxScrollExtent : fallback;
  }
}
