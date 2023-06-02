import 'package:core_domain/core_domain.dart';

extension UriFlowEventExtension on UriFlowEvent {
  void when({
    Function(UriFlowEventDigidAuth)? onDigidAuth,
  }) {
    if (this is UriFlowEventDigidAuth) onDigidAuth?.call(this as UriFlowEventDigidAuth);
  }
}
