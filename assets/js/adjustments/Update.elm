module Adjustments.Update exposing (adjustmentUrl, adjustmentsUrl, fetchAdjustments, put, refreshAdjustmentsTask, saveAdjustmentAndRefreshAdjustments, saveAdjustmentTask, update)

import Adjustments.Messages
import Adjustments.Models exposing (Adjustment)
import Adjustments.Utils exposing (adjustmentDecoder, adjustmentUpdatedDecoder, adjustmentsDecoder, encode)
import Adjustments.Views
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Task exposing (Task)
import Url.Builder as Url


update : Adjustments.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case model.activeAdjustment of
        Just oldActiveAdjustment ->
            case msg of
                Adjustments.Messages.TitleUpdated newName ->
                    let
                        newActiveAdjustment =
                            { oldActiveAdjustment | title = newName }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Adjustments.Messages.AmountUpdated newAmount ->
                    let
                        newActiveAdjustment =
                            case String.toFloat newAmount of
                                Just val ->
                                    { oldActiveAdjustment | amount = val }

                                Nothing ->
                                    { oldActiveAdjustment | amount = 0 }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Adjustments.Messages.SaveAdjustment ->
                    ( model, saveAdjustmentAndRefreshAdjustments oldActiveAdjustment )

                Adjustments.Messages.DeleteAdjustment ->
                    ( model, deleteAdjustmentAndRefreshAdjustments oldActiveAdjustment )

        Nothing ->
            ( model, Cmd.none )


fetchAdjustments : Cmd Msg
fetchAdjustments =
    Http.send AdjustmentsFetched (get adjustmentsUrl adjustmentsDecoder)


deleteAdjustmentAndRefreshAdjustments : Adjustments.Models.Adjustment -> Cmd Msg
deleteAdjustmentAndRefreshAdjustments model =
    Task.attempt AdjustmentsFetched (Task.andThen refreshAdjustmentsTask (deleteAdjustmentTask model))


saveAdjustmentAndRefreshAdjustments : Adjustments.Models.Adjustment -> Cmd Msg
saveAdjustmentAndRefreshAdjustments model =
    case model.id of
        0 ->
            Task.attempt AdjustmentsFetched (Task.andThen refreshAdjustmentsTask (saveNewAdjustmentTask model))

        _ ->
            Task.attempt AdjustmentsFetched (Task.andThen refreshAdjustmentsTask (saveAdjustmentTask model))


refreshAdjustmentsTask : a -> Task Http.Error (List Adjustment)
refreshAdjustmentsTask _ =
    toTask (get adjustmentsUrl adjustmentsDecoder)


saveAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error Adjustment
saveAdjustmentTask model =
    toTask (put (adjustmentUrl model.id) (jsonBody (encode model)) adjustmentUpdatedDecoder)


deleteAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error String
deleteAdjustmentTask model =
    toTask (delete (adjustmentUrl model.id))


saveNewAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error Adjustment
saveNewAdjustmentTask model =
    toTask (post adjustmentsUrl (jsonBody (encode model)) adjustmentUpdatedDecoder)


adjustmentsUrl : String
adjustmentsUrl =
    Url.crossOrigin "//localhost:4000"
        [ "api", "adjustments" ]
        []


adjustmentUrl : Int -> String
adjustmentUrl id =
    Url.crossOrigin "//localhost:4000"
        [ "api", "adjustments", String.fromInt id ]
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
