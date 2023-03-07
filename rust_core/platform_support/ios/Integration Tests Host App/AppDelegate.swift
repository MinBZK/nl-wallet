//
//  AppDelegate.swift
//  Integration Tests Host App
//
//  Created by Wallet Developer on 24/02/2023.
//

import UIKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(_: UIApplication,
                     didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        let window = UIWindow()
        window.backgroundColor = .white
        window.rootViewController = ViewController()
        window.makeKeyAndVisible()

        self.window = window

        return true
    }
}
