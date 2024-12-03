import SwiftUI
import GoogleSignIn

class UserModel: ObservableObject {
  @Published var idToken: String?
}

@main
struct SimpleBudget: App {
  @StateObject var userModel: UserModel = .init()
  
  var body: some Scene {
    WindowGroup {
      ContentView()
      .onOpenURL(perform: { (url: URL) in
        GIDSignIn.sharedInstance.handle(url)
      }).onAppear(perform: {
        GIDSignIn.sharedInstance.restorePreviousSignIn { user, error in
          guard error == nil else { return }
          print("here")
          userModel.idToken = user?.idToken?.tokenString
        }
      })
      .environmentObject(userModel)
    }
  }
}

