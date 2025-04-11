import Foundation

enum APIError: Error {
    case invalidURL
    case requestFailed(Error)
    case invalidResponse
    case decodingFailed(Error)
    case unauthorized
    case serverError(Int)
    case unknown
    
    var message: String {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .requestFailed(let error):
            return "Request failed: \(error.localizedDescription)"
        case .invalidResponse:
            return "Invalid response from server"
        case .decodingFailed(let error):
            return "Failed to decode response: \(error.localizedDescription)"
        case .unauthorized:
            return "Unauthorized access"
        case .serverError(let code):
            return "Server error with code: \(code)"
        case .unknown:
            return "Unknown error occurred"
        }
    }
}

class APIService {
    private let baseURL: String
    private let session: URLSession
    
    init(baseURL: String = "http://localhost:8000", session: URLSession = .shared) {
        self.baseURL = baseURL
        self.session = session
    }
    
    // Generic request method that handles different HTTP methods and response types
    func request<T: Decodable>(
        endpoint: String,
        method: String = "GET",
        body: Data? = nil,
        headers: [String: String]? = nil,
        idToken: String? = nil
    ) async throws -> T {
        guard let url = URL(string: "\(baseURL)/\(endpoint)") else {
            throw APIError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        // Add authorization header if ID token is provided
        if let idToken = idToken {
            request.setValue("Bearer \(idToken)", forHTTPHeaderField: "Authorization")
        }
        
        // Add any additional headers
        headers?.forEach { key, value in
            request.setValue(value, forHTTPHeaderField: key)
        }
        
        // Add body data if provided
        if let body = body {
            request.httpBody = body
        }
        
        do {
            let (data, response) = try await session.data(for: request)
            
            guard let httpResponse = response as? HTTPURLResponse else {
                throw APIError.invalidResponse
            }
            
            switch httpResponse.statusCode {
            case 200...299:
                do {
                    return try JSONDecoder().decode(T.self, from: data)
                } catch let error {
                    throw APIError.decodingFailed(error)
                }
            case 401:
                throw APIError.unauthorized
            case 400...499:
                throw APIError.serverError(httpResponse.statusCode)
            case 500...599:
                throw APIError.serverError(httpResponse.statusCode)
            default:
                throw APIError.unknown
            }
        } catch let error as APIError {
            throw error
        } catch let error {
            throw APIError.requestFailed(error)
        }
    }
    
    // Convenience method for POST requests with Encodable body
    func post<T: Decodable, E: Encodable>(
        endpoint: String,
        body: E,
        idToken: String? = nil
    ) async throws -> T {
        let bodyData = try JSONEncoder().encode(body)
        return try await request(
            endpoint: endpoint,
            method: "POST",
            body: bodyData,
            idToken: idToken
        )
    }
    
    // Convenience method for GET requests
    func get<T: Decodable>(
        endpoint: String,
        idToken: String? = nil
    ) async throws -> T {
        return try await request(
            endpoint: endpoint,
            method: "GET",
            idToken: idToken
        )
    }
    
    // Method specifically for authentication token exchange
    func exchangeToken(idToken: String) async throws -> String {
        struct TokenRequest: Encodable {
            let id_token: String
        }
        
        struct TokenResponse: Decodable {
            let token: String
        }
        
        let response: TokenResponse = try await post(
            endpoint: "authentication/token",
            body: TokenRequest(id_token: idToken)
        )
        
        return response.token
    }
}
