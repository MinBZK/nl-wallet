extension NumExtensions on num {
  num normalize(double min, double max) => (this - min) / (max - min);
}
