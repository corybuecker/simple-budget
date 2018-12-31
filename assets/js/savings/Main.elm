module Main exposing (errorMessage, init, main, subscriptions, update, view)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http exposing (get, jsonBody, post, toTask)
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

        Just (Http.BadPayload message _) ->
            message

        _ ->
            "Unknown"


deleteSavingAndRefreshSaving : Saving -> Cmd Msg
deleteSavingAndRefreshSaving model =
    Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (deleteSavingTask model))


fetchSavings : Cmd Msg
fetchSavings =
    Http.send SavingsFetched (get savingsUrl savingsDecoder)


saveSavingAndRefreshSavings : Saving -> Cmd Msg
saveSavingAndRefreshSavings model =
    case model.id of
        0 ->
            Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (saveNewSavingTask model))

        _ ->
            Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (saveSavingTask model))


saveNewSavingTask : Saving -> Task Http.Error Saving
saveNewSavingTask model =
    toTask (post savingsUrl (jsonBody (Savings.Utils.encode model)) savingUpdatedDecoder)


refreshSavingsTask : a -> Task Http.Error (List Saving)
refreshSavingsTask _ =
    toTask (get savingsUrl savingsDecoder)


saveSavingTask : Saving -> Task Http.Error Saving
saveSavingTask model =
    toTask (put (savingUrl model.id) (jsonBody (Savings.Utils.encode model)) savingUpdatedDecoder)


deleteSavingTask : Saving -> Task Http.Error String
deleteSavingTask model =
    toTask (delete (savingUrl model.id))


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


delete : String -> Http.Request String
delete url =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectString
        , timeout = Nothing
        , withCredentials = False
        }
