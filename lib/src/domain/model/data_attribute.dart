import 'package:equatable/equatable.dart';

class DataAttribute extends Equatable {
  final DataAttributeType type;
  final String label;
  final String? value;
  final DataAttributeValueType valueType;

  const DataAttribute({
    required this.valueType,
    required this.label,
    required this.value,
    this.type = DataAttributeType.other,
  });

  @override
  List<Object?> get props => [valueType, label, value, type];
}

enum DataAttributeValueType { image, text }

enum DataAttributeType {
  firstNames,
  lastName,
  fullName,
  gender,
  profilePhoto,
  birthDate,
  birthPlace,
  birthCountry,
  citizenshipNumber,
  documentNr,
  issuanceDate,
  expiryDate,
  height,
  university,
  education,
  educationLevel,
  certificateOfConduct,
  phone,
  email,
  address,
  olderThan18,
  healthIssuerId,
  other,
}
