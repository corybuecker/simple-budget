module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http exposing (get, jsonBody, post)
import Json.Decode
import Json.Decode.Pipeline exposing (hardcoded, optional, required)
import List exposing (..)
import Savings.Messages exposing (..)
import Savings.Models exposing (..)
import Savings.Utils exposing (..)
import Savings.Views exposing (..)
import String
import Task exposing (Task)
import Url exposing (Url)
import Url.Builder as Url


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( emptyModel
    , fetchSavings
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SavingsFetched result ->
            case result of
                Ok savings ->
                    ( { model | savings = savings, activeSaving = Nothing }
                    , Cmd.none
                    )

                Err error ->
                    ( { model | error = Just error }
                    , Cmd.none
                    )

        TitleUpdated newName ->
            case model.activeSaving of
                Just oldActiveSaving ->
                    let
                        newActiveSaving =
                            { oldActiveSaving | title = newName }
                    in
                    ( { model | activeSaving = Just newActiveSaving }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        AmountUpdated newAmount ->
            case model.activeSaving of
                Just oldActiveSaving ->
                    let
                        newActiveSaving =
                            case String.toFloat newAmount of
                                Just val ->
                                    { oldActiveSaving | amount = val }

                                Nothing ->
                                    { oldActiveSaving | amount = 0 }
                    in
                    ( { model | activeSaving = Just newActiveSaving }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        OpenSavingEditor saving ->
            ( { model | modalOpen = "saving", activeSaving = Just saving }, Cmd.none )

        CreateSaving ->
            ( { model | activeSaving = Just Savings.Models.newSaving }, Cmd.none )

        SaveSaving ->
            case model.activeSaving of
                Just oldActiveSaving ->
                    ( model, saveSavingAndRefreshSavings oldActiveSaving )

                Nothing ->
                    ( model, Cmd.none )

        DeleteSaving ->
            case model.activeSaving of
                Just oldActiveSaving ->
                    ( model, deleteSavingAndRefreshSaving oldActiveSaving )

                Nothing ->
                    ( model, Cmd.none )

        DeleteCompleted result ->
            ( model, fetchSavings )

        SavingSaved result ->
            ( model, fetchSavings )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


modalView : Model -> Html Msg
modalView model =
    case model.activeSaving of
        Just a ->
            editView a

        Nothing ->
            div [] []


view : Model -> Html Msg
view model =
    div []
        [ div [] [ renderSavings model.savings ]
        , div [] [ modalView model ]
        , p [] [ text (errorMessage model.error) ]
        ]


errorMessage : Maybe Http.Error -> String
errorMessage error =
    case error of
        Nothing ->
            ""

        Just (Http.BadBody message) ->
            message

        _ ->
            "Unknown"


deleteSavingAndRefreshSaving : Saving -> Cmd Msg
deleteSavingAndRefreshSaving model =
    deleteSavingTask model


fetchSavings : Cmd Msg
fetchSavings =
    get savingsUrl SavingsFetched savingsDecoder


saveSavingAndRefreshSavings : Saving -> Cmd Msg
saveSavingAndRefreshSavings model =
    case model.id of
        0 ->
            saveNewSavingTask model

        _ ->
            saveSavingTask model


saveNewSavingTask : Saving -> Cmd Msg
saveNewSavingTask model =
    post savingsUrl (jsonBody (Savings.Utils.encode model)) SavingSaved savingUpdatedDecoder


refreshSavingsTask : a -> Cmd Msg
refreshSavingsTask _ =
    get savingsUrl SavingsFetched savingsDecoder


saveSavingTask : Saving -> Cmd Msg
saveSavingTask model =
    put (savingUrl model.id) (jsonBody (Savings.Utils.encode model)) SavingSaved savingUpdatedDecoder


deleteSavingTask : Saving -> Cmd Msg
deleteSavingTask model =
    delete (savingUrl model.id)


savingsUrl : String
savingsUrl =
    Url.relative
        [ "api", "savings" ]
        []


savingUrl : Int -> String
savingUrl id =
    Url.relative
        [ "api", "savings", String.fromInt id ]
        []


get : String -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
get url msg decoder =
    Http.request
        { method = "GET"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


put : String -> Http.Body -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
put url body msg decoder =
    Http.request
        { method = "PUT"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


post : String -> Http.Body -> (Result Http.Error a -> Msg) -> Json.Decode.Decoder a -> Cmd Msg
post url body msg decoder =
    Http.request
        { method = "POST"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson msg decoder
        , timeout = Nothing
        , tracker = Nothing
        }


delete : String -> Cmd Msg
delete url =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectWhatever DeleteCompleted
        , timeout = Nothing
        , tracker = Nothing
        }
