class CardFront {
  final String title;
  final String? subtitle;
  final String? info;
  final String? logoImage;
  final String? backgroundImage;
  final CardFrontTheme theme;

  const CardFront({
    required this.title,
    this.subtitle,
    this.info,
    this.logoImage,
    this.backgroundImage,
    required this.theme,
  });
}

enum CardFrontTheme {
  light, // light background + dark texts
  dark, // dark background + light texts
}
