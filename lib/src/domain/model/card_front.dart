class CardFront {
  final String title;
  final String? subtitle;
  final String? info;
  final String? logoImage;
  final String? backgroundImage;

  const CardFront({
    required this.title,
    this.subtitle,
    this.info,
    this.logoImage,
    this.backgroundImage,
  });
}
