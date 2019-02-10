module Main exposing (Model)

import Browser
import Browser.Navigation
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import Json.Encode


type alias Model =
    { username : String
    , password : String
    , token : String
    , invalid : Bool
    }


type Msg
    = UsernameUpdated String
    | PasswordUpdated String
    | Login
    | TokenFetched (Result Http.Error String)
    | LoggedIn (Result Http.Error ())
    | NoOperation


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model "" "" "" False, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        UsernameUpdated username ->
            ( { model | username = username, invalid = False }, Cmd.none )

        PasswordUpdated password ->
            ( { model | password = password, invalid = False }, Cmd.none )

        Login ->
            ( model, createToken model )

        TokenFetched result ->
            case result of
                Ok token ->
                    let
                        newModel =
                            { model | token = token }
                    in
                    ( newModel, createLogin newModel )

                Err _ ->
                    ( { model | token = "", invalid = True }, Cmd.none )

        LoggedIn result ->
            case result of
                Ok _ ->
                    ( model, Browser.Navigation.load "/accounts" )

                Err _ ->
                    ( { model | invalid = True }, Cmd.none )

        NoOperation ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


view : Model -> Html Msg
view model =
    div []
        [ Html.form [ onSubmit NoOperation ]
            [ div [ class "input-wrapper" ]
                [ label [ for "email" ] [ text "Email" ]
                , input
                    [ attribute "aria-label" "Email"
                    , property "type" (Json.Encode.string "email")
                    , name "email"
                    , id "email"
                    , value model.username
                    , onInput UsernameUpdated
                    ]
                    []
                ]
            , div [ class "input-wrapper" ]
                [ label [ for "password" ] [ text "Password" ]
                , input
                    [ attribute "aria-label" "Password"
                    , property "type" (Json.Encode.string "password")
                    , name "password"
                    , id "password"
                    , value model.password
                    , onInput PasswordUpdated
                    ]
                    []
                ]
            , button
                [ property "type" (Json.Encode.string "submit")
                , onClick Login
                ]
                [ text "Log in" ]
            , showErrorStatus model
            ]
        ]


showErrorStatus : Model -> Html Msg
showErrorStatus model =
    case model.invalid of
        True ->
            div [] [ text "Cannot authenticate" ]

        False ->
            div [] []


createLogin : Model -> Cmd Msg
createLogin model =
    let
        login =
            Json.Encode.object
                [ ( "idtoken", Json.Encode.string model.token )
                ]
    in
    Http.request
        { method = "POST"
        , headers = []
        , url = "/login"
        , body = Http.jsonBody login
        , expect = Http.expectWhatever LoggedIn
        , timeout = Nothing
        , tracker = Nothing
        }


createToken : Model -> Cmd Msg
createToken model =
    let
        login =
            Json.Encode.object
                [ ( "email", Json.Encode.string model.username )
                , ( "password", Json.Encode.string model.password )
                ]
    in
    Http.request
        { method = "POST"
        , headers = []
        , url = "/token"
        , body = Http.jsonBody login
        , expect = Http.expectJson TokenFetched tokenDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


tokenDecoder : Json.Decode.Decoder String
tokenDecoder =
    Json.Decode.field "idtoken" Json.Decode.string
