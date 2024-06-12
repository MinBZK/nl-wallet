extension NumExtensions on num {
  num normalize(num min, num max) {
    final num result = clamp(min, max);
    return (result - min) / (max - min);
  }
}
