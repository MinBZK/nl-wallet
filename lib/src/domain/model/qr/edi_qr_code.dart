class EdiQrCode {
  final int id;

  EdiQrCode({required this.id});

  factory EdiQrCode.fromJson(Map<String, dynamic> json) {
    return EdiQrCode(id: json['id']);
  }

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{};
    map['id'] = id;
    return map;
  }
}
