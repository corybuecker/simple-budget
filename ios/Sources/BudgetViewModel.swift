import Foundation
import SwiftUI
import Combine

// Main view model that handles all budget-related API calls
class BudgetViewModel: ObservableObject {
    // Published properties for UI binding
    @Published var accounts: [Account] = []
    @Published var envelopes: [Envelope] = []
    @Published var goals: [Goal] = []
    @Published var isLoading: Bool = false
    @Published var error: String? = nil
    
    // API service
    private let apiService: APIService
    private var cancellables = Set<AnyCancellable>()
    
    // User model for authentication
    @Published var userModel: UserModel
    private var authToken: String? = nil
    
    init(apiService: APIService = APIService(), userModel: UserModel) {
        self.apiService = apiService
        self.userModel = userModel
        
        // Subscribe to changes in the user's ID token
        userModel.$idToken
            .sink { [weak self] idToken in
                if let idToken = idToken {
                    self?.authenticateWithBackend(idToken: idToken)
                } else {
                    // Clear data when user logs out
                    self?.clearData()
                }
            }
            .store(in: &cancellables)
    }
    
    // MARK: - Authentication
    
    private func authenticateWithBackend(idToken: String) {
        isLoading = true
        error = nil
        
        Task {
            do {
                let token = try await apiService.exchangeToken(idToken: idToken)
                await MainActor.run {
                    self.authToken = token
                    self.isLoading = false
                    // Load initial data after authentication
                    self.loadAllData()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    private func clearData() {
        authToken = nil
        accounts = []
        envelopes = []
        goals = []
    }
    
    // MARK: - Data Loading
    
    func loadAllData() {
        loadAccounts()
        loadEnvelopes()
        loadGoals()
    }
    
    // MARK: - Account Methods
    
    func loadAccounts() {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let fetchedAccounts: [Account] = try await apiService.get(
                    endpoint: "authenticated/accounts",
                    idToken: token
                )
                
                await MainActor.run {
                    self.accounts = fetchedAccounts
                    self.isLoading = false
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func createAccount(name: String, balance: Decimal) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = CreateAccountRequest(name: name, balance: balance)
        
        Task {
            do {
                let _: Account = try await apiService.post(
                    endpoint: "authenticated/accounts/create",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadAccounts()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func updateAccount(id: Int, name: String, balance: Decimal) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = UpdateAccountRequest(name: name, balance: balance)
        
        Task {
            do {
                let _: Account = try await apiService.post(
                    endpoint: "authenticated/accounts/\(id)/update",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadAccounts()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func deleteAccount(id: Int) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let _: EmptyResponse = try await apiService.post(
                    endpoint: "authenticated/accounts/\(id)/delete",
                    body: EmptyRequest(),
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadAccounts()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    // MARK: - Envelope Methods
    
    func loadEnvelopes() {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let fetchedEnvelopes: [Envelope] = try await apiService.get(
                    endpoint: "authenticated/envelopes",
                    idToken: token
                )
                
                await MainActor.run {
                    self.envelopes = fetchedEnvelopes
                    self.isLoading = false
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func createEnvelope(name: String, amount: Decimal) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = CreateEnvelopeRequest(name: name, amount: amount)
        
        Task {
            do {
                let _: Envelope = try await apiService.post(
                    endpoint: "authenticated/envelopes/create",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadEnvelopes()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func updateEnvelope(id: Int, name: String, amount: Decimal) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = UpdateEnvelopeRequest(name: name, amount: amount)
        
        Task {
            do {
                let _: Envelope = try await apiService.post(
                    endpoint: "authenticated/envelopes/\(id)/update",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadEnvelopes()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func deleteEnvelope(id: Int) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let _: EmptyResponse = try await apiService.post(
                    endpoint: "authenticated/envelopes/\(id)/delete",
                    body: EmptyRequest(),
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadEnvelopes()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    // MARK: - Goal Methods
    
    func loadGoals() {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let fetchedGoals: [Goal] = try await apiService.get(
                    endpoint: "authenticated/goals",
                    idToken: token
                )
                
                await MainActor.run {
                    self.goals = fetchedGoals
                    self.isLoading = false
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func createGoal(name: String, targetAmount: Decimal, targetDate: Date? = nil) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = CreateGoalRequest(name: name, targetAmount: targetAmount, targetDate: targetDate)
        
        Task {
            do {
                let _: Goal = try await apiService.post(
                    endpoint: "authenticated/goals/create",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadGoals()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func updateGoal(id: Int, name: String, targetAmount: Decimal, accumulatedAmount: Decimal, targetDate: Date? = nil) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        let request = UpdateGoalRequest(name: name, targetAmount: targetAmount, accumulatedAmount: accumulatedAmount, targetDate: targetDate)
        
        Task {
            do {
                let _: Goal = try await apiService.post(
                    endpoint: "authenticated/goals/\(id)/update",
                    body: request,
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadGoals()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    func deleteGoal(id: Int) {
        guard let token = authToken else {
            error = "Not authenticated"
            return
        }
        
        isLoading = true
        error = nil
        
        Task {
            do {
                let _: EmptyResponse = try await apiService.post(
                    endpoint: "authenticated/goals/\(id)/delete",
                    body: EmptyRequest(),
                    idToken: token
                )
                
                await MainActor.run {
                    self.isLoading = false
                    self.loadGoals()
                }
            } catch let apiError as APIError {
                await MainActor.run {
                    self.error = apiError.message
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.error = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
}

// Helper empty request/response for endpoints that don't need data
struct EmptyRequest: Encodable {}
struct EmptyResponse: Decodable {}
