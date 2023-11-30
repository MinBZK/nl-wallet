import 'package:wallet_core/core.dart';

import 'requested_attribute.dart';

class SignRequest {
  final String id;
  final Organization organization;
  final Organization trustProvider;
  final Document document;
  final List<RequestedAttribute> requestedAttributes;
  final RequestPolicy policy;

  const SignRequest({
    required this.id,
    required this.organization,
    required this.document,
    required this.trustProvider,
    required this.requestedAttributes,
    required this.policy,
  });
}

class Document {
  final String title;
  final String fileName;
  final String url;

  const Document({
    required this.title,
    required this.fileName,
    required this.url,
  });
}
