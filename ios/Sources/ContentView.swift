//
//  ContentView.swift
//  simple-budget
//
//  Created by Cory Buecker on 10/11/24.
//


import SwiftUI
import GoogleSignIn
import GoogleSignInSwift

struct ContentView: View {
  var body: some View {
    SignInView()
    GoogleSignInButton(action: handleSignInButton(view: self))
  }
}

struct SignInView: View {
  var body: some View {
    Text("Hello, World!")
  }
}

func handleSignInButton(view: ContentView) -> () -> Void {
  return {
    GIDSignIn.sharedInstance.signIn(withPresenting: UIHostingController(rootView: view))
  }
}
