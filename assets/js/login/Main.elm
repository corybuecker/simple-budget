module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import Json.Encode
import Task exposing (Task)
import Url exposing (Url)
import Url.Builder as Url


type alias Model =
    { useDummy : Bool
    , username : String
    , password : String
    }


type Msg
    = UsernameUpdated String
    | PasswordUpdated String
    | Login
    | NoOperation


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : Bool -> ( Model, Cmd Msg )
init useDummy =
    ( Model useDummy "" "", Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        UsernameUpdated username ->
            ( { model | username = username }, Cmd.none )

        PasswordUpdated password ->
            ( { model | password = password }, Cmd.none )

        Login ->
            ( model, Cmd.none )

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
                    [ property "type" (Json.Encode.string "password")
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
            ]
        ]


errorMessage : Maybe Http.Error -> String
errorMessage error =
    case error of
        Nothing ->
            ""

        Just (Http.BadPayload message _) ->
            message

        _ ->
            "Unknown"


put : String -> Http.Body -> Json.Decode.Decoder a -> Http.Request a
put url body decoder =
    Http.request
        { method = "PUT"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson decoder
        , timeout = Nothing
        , withCredentials = False
        }
