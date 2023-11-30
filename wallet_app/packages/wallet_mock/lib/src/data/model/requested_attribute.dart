/// Simple Attribute used in mock requests, the key is used to check if the attribute is available in the user's
/// wallet. The label is rendered to the screen when the attribute is missing and this needs to be communicated.
class RequestedAttribute {
  final String key;
  final String label;

  RequestedAttribute({required this.key, required this.label});
}
