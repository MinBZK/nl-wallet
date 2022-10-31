class WalletCard {
  final String id;
  final String title;
  final String? subtitle;
  final String? info;
  final String? logoImage;
  final String? backgroundImage;

  const WalletCard({
    required this.id,
    required this.title,
    this.subtitle,
    this.info,
    this.logoImage,
    this.backgroundImage,
  });
}
