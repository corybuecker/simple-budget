module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser exposing (element)
import Html exposing (Html, div, p, text)
import Http exposing (Error, expectJson, get)
import Json.Decode exposing (Decoder, field, float, map2)


type alias CalculationResponse =
    { remaining : Float
    , remainingPerDay : Float
    }


type alias Model =
    { remaining : Float
    , remainingPerDay : Float
    , error : Maybe Http.Error
    }


type Msg
    = CalculationsFetched (Result Http.Error CalculationResponse)


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model 0.0 0.0 Nothing
    , fetchCalculations
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CalculationsFetched result ->
            case result of
                Ok response ->
                    ( { model | remainingPerDay = response.remainingPerDay, remaining = response.remaining }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


view : Model -> Html Msg
view model =
    div []
        [ div [] [ text (String.fromFloat model.remaining) ]
        , div [] [ text (String.fromFloat model.remainingPerDay) ]
        , p [] [ text (errorMessage model.error) ]
        ]


fetchCalculations : Cmd Msg
fetchCalculations =
    get
        { url = "/api/calculations"
        , expect = expectJson CalculationsFetched calculationsDecoder
        }


calculationsDecoder : Decoder CalculationResponse
calculationsDecoder =
    map2 CalculationResponse
        remainingDecoder
        remainingPerDayDecoder


remainingDecoder : Decoder Float
remainingDecoder =
    field "data" (field "remaining" float)


remainingPerDayDecoder : Decoder Float
remainingPerDayDecoder =
    field "data" (field "remaining_per_day" float)


errorMessage : Maybe Error -> String
errorMessage error =
    case error of
        Nothing ->
            ""

        Just (Http.BadBody message) ->
            message

        _ ->
            "Unknown"
