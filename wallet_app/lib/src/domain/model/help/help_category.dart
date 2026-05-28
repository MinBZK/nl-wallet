import 'package:equatable/equatable.dart';

import 'help_subcategory.dart';

class HelpCategory extends Equatable {
  final String id;
  final String icon;
  final String title;
  final String subtitle;
  final List<HelpSubcategory> subcategories;

  const HelpCategory({
    required this.id,
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.subcategories,
  });

  @override
  List<Object?> get props => [id, icon, title, subtitle, subcategories];
}
