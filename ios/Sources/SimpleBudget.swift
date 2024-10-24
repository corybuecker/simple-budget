import SwiftUI
import GoogleSignIn

class UserModel: ObservableObject {
  @Published var name: String?
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
          
          userModel.name = user?.profile?.name
        }
      })
      .environmentObject(userModel)
    }
  }
}

