module Savings.Update exposing (fetchSavings, put, refreshSavingsTask, saveSavingAndRefreshSavings, saveSavingTask, savingUrl, savingsUrl, update)

import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Savings.Messages
import Savings.Models exposing (Saving)
import Savings.Utils exposing (encode, savingDecoder, savingUpdatedDecoder, savingsDecoder)
import Savings.Views
import Task exposing (Task)
import Url.Builder as Url


update : Savings.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case model.activeSaving of
        Just oldActiveSaving ->
            case msg of
                Savings.Messages.TitleUpdated newName ->
                    let
                        newActiveSaving =
                            { oldActiveSaving | title = newName }
                    in
                    ( { model | activeSaving = Just newActiveSaving }, Cmd.none )

                Savings.Messages.AmountUpdated newAmount ->
                    let
                        newActiveSaving =
                            case String.toFloat newAmount of
                                Just val ->
                                    { oldActiveSaving | amount = val }

                                Nothing ->
                                    { oldActiveSaving | amount = 0 }
                    in
                    ( { model | activeSaving = Just newActiveSaving }, Cmd.none )

                Savings.Messages.SaveSaving ->
                    ( model, saveSavingAndRefreshSavings oldActiveSaving )

                Savings.Messages.DeleteSaving ->
                    ( model, deleteSavingAndRefreshSavings oldActiveSaving )

        Nothing ->
            ( model, Cmd.none )


fetchSavings : Cmd Msg
fetchSavings =
    Http.send SavingsFetched (get savingsUrl savingsDecoder)


deleteSavingAndRefreshSavings : Savings.Models.Saving -> Cmd Msg
deleteSavingAndRefreshSavings model =
    Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (deleteSavingTask model))


saveSavingAndRefreshSavings : Savings.Models.Saving -> Cmd Msg
saveSavingAndRefreshSavings model =
    case model.id of
        0 ->
            Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (saveNewSavingTask model))

        _ ->
            Task.attempt SavingsFetched (Task.andThen refreshSavingsTask (saveSavingTask model))


refreshSavingsTask : a -> Task Http.Error (List Saving)
refreshSavingsTask _ =
    toTask (get savingsUrl savingsDecoder)


saveSavingTask : Savings.Models.Saving -> Task Http.Error Saving
saveSavingTask model =
    toTask (put (savingUrl model.id) (jsonBody (encode model)) savingUpdatedDecoder)


deleteSavingTask : Savings.Models.Saving -> Task Http.Error String
deleteSavingTask model =
    toTask (delete (savingUrl model.id))


saveNewSavingTask : Savings.Models.Saving -> Task Http.Error Saving
saveNewSavingTask model =
    toTask (post savingsUrl (jsonBody (encode model)) savingUpdatedDecoder)


savingsUrl : String
savingsUrl =
    Url.crossOrigin "//localhost:4000"
        [ "api", "savings" ]
        []


savingUrl : Int -> String
savingUrl id =
    Url.crossOrigin "//localhost:4000"
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
